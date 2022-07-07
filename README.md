# Stop Family

> This was just a fun project to learn the Windows Task Scheduler API and file watching. You shouldn't use this project if you really want to stop Windows' Family Features. If you have administrator privileges, you should probably follow the solutions in this video: https://youtu.be/MaSILURLVK0?t=40

This program allows you to circumvent all of the restrictions of Windows' Family Features.

You are able to use blocked apps while using this.
If you want to access a game on the Xbox app thats blocked, you must first find the game on the Microsoft store
and then you will be able to install the game.

## The history of the program.

At first, this program was just a simple program that just killed `WPCMon.exe`.
That approach was flawed because, while it was able to circumvent the time limit,
it was not able to circumvent the app restrictions.

next, I tried to just delete `WPCMon.exe`. To do that I had to gain the privileges to delete files in System32, but then I noticed that it caused system instability and wanted to use a different method.

then, after a little research, I found that the `ms-wpc://` URI existed, and that maybe I could use it. (see [#1](https://github.com/TheBotlyNoob/Stop-Family/issues/1)), but then I found that the URI had nothing to do with killing the program, it just showed the popups.

Finally after looking at a few other methods, such as deleting the AppxPackage (`Microsoft.Windows.ParentalControls_1000.22000.1.0_neutral_neutral_cw5n1h2txyewy`), that's a whole 'nother can of worms (see the [super user post](https://superuser.com/questions/1115801/unable-to-uninstall-universal-apps-through-powershell) and the [blog post](https://www.winhelponline.com/blog/error-0x80073cfa-uninstall-app-removal-failed/) about it). I found the `C:\ProgramData\Microsoft\Windows\Parental Controls\settings` directory, which contained `.bin` files that were actually JSON. I found that if I delete them, Windows wouldn't think that parental controls were enabled, but they regenerate after a bit of time or a reboot.
