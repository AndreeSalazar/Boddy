use boddy_core::{Error, Mission, Policy, Stage, Task, Tool, MAX_EVIDENCE};

fn task(limit: u16) -> Task<'static> {
    Task::new(
        Mission::new("Ayudar al dueño", "Revisar un proyecto", "Pruebas pasan").unwrap(),
        Policy::local_research(),
        limit,
    )
    .unwrap()
}

fn planned() -> Task<'static> {
    let mut t = task(8);
    t.start().unwrap();
    t.record_evidence("SG:resumen").unwrap();
    t.set_plan("plan:1").unwrap();
    t
}

#[test]
fn requires_reason_objective_and_acceptance() {
    for fields in [(" ", "b", "c"), ("a", "", "c"), ("a", "b", "\n")] {
        assert_eq!(
            Mission::new(fields.0, fields.1, fields.2),
            Err(Error::EmptyField)
        );
    }
    assert_eq!(
        Task::new(
            Mission::new("a", "b", "c").unwrap(),
            Policy::local_research(),
            0
        )
        .unwrap_err(),
        Error::InvalidBudget
    );
}

#[test]
fn cannot_plan_without_context() {
    let mut t = task(8);
    t.start().unwrap();
    assert_eq!(t.set_plan("plan:1"), Err(Error::MissingEvidence));
    assert_eq!(t.stage(), Stage::Researching);
}

#[test]
fn baseline_denies_execution_and_network_without_spending_budget() {
    let mut t = planned();
    assert_eq!(t.request_tool(Tool::ExecuteCode), Err(Error::ToolDenied));
    assert_eq!(t.request_tool(Tool::Network), Err(Error::ToolDenied));
    assert_eq!(t.tool_calls(), 0);
    t.request_tool(Tool::ProposePatch).unwrap();
    assert_eq!(t.tool_calls(), 1);
}

#[test]
fn tool_budget_cannot_wrap_or_be_reused() {
    let mut t = task(u16::MAX);
    t.start().unwrap();
    for _ in 0..u16::MAX {
        t.request_tool(Tool::ReadGraph).unwrap();
    }
    assert_eq!(t.request_tool(Tool::ReadGraph), Err(Error::BudgetExhausted));
    assert_eq!(t.tool_calls(), u16::MAX);
}

#[test]
fn evidence_is_bounded_and_duplicates_do_not_consume_slots() {
    let mut t = task(8);
    t.start().unwrap();
    for s in ["a", "b", "c", "d", "e", "f", "g", "h"] {
        t.record_evidence(s).unwrap();
    }
    t.record_evidence(" a ").unwrap();
    assert_eq!(t.evidence_count(), MAX_EVIDENCE);
    assert_eq!(t.record_evidence("i"), Err(Error::EvidenceFull));
    assert_eq!(t.record_evidence(" "), Err(Error::EmptyField));
}

#[test]
fn success_requires_a_host_verification_artifact() {
    let mut t = planned();
    assert_eq!(t.complete(), Err(Error::InvalidStage));
    t.begin_verification().unwrap();
    assert_eq!(t.complete(), Err(Error::NotVerified));
    assert_eq!(t.record_verification(true, ""), Err(Error::EmptyField));
    t.record_verification(false, "tests:failed").unwrap();
    assert_eq!(t.complete(), Err(Error::NotVerified));
    t.record_verification(true, "tests:passed").unwrap();
    t.complete().unwrap();
    assert_eq!(t.stage(), Stage::Completed);
}

#[test]
fn revision_invalidates_old_verification() {
    let mut t = planned();
    t.begin_verification().unwrap();
    t.record_verification(true, "old:passed").unwrap();
    t.revise().unwrap();
    assert_eq!(t.verification_reference(), None);
    t.begin_verification().unwrap();
    assert_eq!(t.complete(), Err(Error::NotVerified));
}

#[test]
fn verification_blocks_further_mutation_and_tool_calls() {
    let mut t = planned();
    t.begin_verification().unwrap();
    assert_eq!(t.request_tool(Tool::ReadGraph), Err(Error::InvalidStage));
    assert_eq!(t.record_evidence("late"), Err(Error::InvalidStage));
    assert_eq!(t.set_plan("late"), Err(Error::InvalidStage));
}

#[test]
fn stopped_task_cannot_resume_or_complete() {
    let mut t = planned();
    t.stop().unwrap();
    assert_eq!(t.start(), Err(Error::InvalidStage));
    assert_eq!(t.request_tool(Tool::ReadGraph), Err(Error::InvalidStage));
    assert_eq!(t.complete(), Err(Error::InvalidStage));
}
