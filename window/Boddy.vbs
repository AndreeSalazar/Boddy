Set shell = CreateObject("WScript.Shell")
Set files = CreateObject("Scripting.FileSystemObject")
project = files.GetParentFolderName(files.GetParentFolderName(WScript.ScriptFullName))
launcher = files.BuildPath(project, "target\debug\boddy-window.exe")
If files.FileExists(launcher) Then
  shell.Run """" & launcher & """", 1, False
Else
  MsgBox "Compila primero: cargo build -p boddy-window --offline", 48, "Boddy 1.0"
End If
