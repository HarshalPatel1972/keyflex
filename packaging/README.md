# Publishing Keyflex to the Microsoft Store

The Store signs the package during certification, so no code-signing
certificate is needed. It also handles installing and updating.

## 1. Reserve the name

In [Partner Center](https://partner.microsoft.com/dashboard), create a new app
and reserve the name **Keyflex**.

## 2. Get the three identity values

In Partner Center, open the app, then **Product management > Product identity**.
Copy:

| Partner Center field | Script parameter | Looks like |
|---|---|---|
| Package/Identity/Name | `-IdentityName` | `12345YourName.Keyflex` |
| Package/Identity/Publisher | `-Publisher` | `CN=1A2B3C4D-5E6F-...` |
| Package/Properties/PublisherDisplayName | `-PublisherDisplayName` | `Your Name` |

They must match exactly, or the upload is rejected.

## 3. Build the package

```powershell
pwsh packaging/build-msix.ps1 `
  -IdentityName "12345YourName.Keyflex" `
  -Publisher "CN=1A2B3C4D-5E6F-..." `
  -PublisherDisplayName "Your Name"
```

The result is `packaging/out/Keyflex_<version>_x64.msix`. Upload that file on
the **Packages** page of the submission.

To raise the version for an update, change `version` in
`src-tauri/tauri.conf.json` (and `package.json`, `src-tauri/Cargo.toml`) and
build again. The Store requires each upload to have a higher version.

## 4. What the submission will ask for

**Restricted capability: `runFullTrust`.** Partner Center asks why it is
needed. Suggested answer:

> Keyflex is a desktop utility that shows keyboard-shortcut tips. It needs full
> trust to place an icon in the notification area, to use UI Automation to read
> the name of the button or menu item the user clicks, and to detect when the
> user presses one of the shortcuts it teaches. It does not record keystrokes
> and makes no network connections.

**Privacy policy URL.** Required, because the app observes input. Use the
address of `PRIVACY.md` on GitHub, or the same text on your own site:

`https://github.com/HarshalPatel1972/keyflex/blob/main/PRIVACY.md`

**Notes for certification.** Tell the tester how to see the app working, since
it is otherwise invisible. Suggested text:

> Keyflex runs in the notification area (system tray). To see a tip: open
> Chrome or Edge, click the "..." menu, then click Downloads. A popup appears
> beside the cursor suggesting Ctrl+J. The Settings screen has a "Show a
> sample" button that shows the popup immediately. Keyflex reads only the names
> of buttons and menu items; it does not log keystrokes, and it sends nothing
> over the network.

**Screenshots.** At least one, 1366 x 768 or larger. The Home screen, the
Shortcuts screen, and a tip popup over a browser make a good set.

**Age rating and category.** Category: Productivity (or Utilities & tools).

## Things that behave differently in the Store version

- **Start with Windows** uses a Windows startup task instead of a registry
  entry. The switch in Keyflex's Settings controls it. If the user turns it off
  in Windows' own Settings > Apps > Startup, only they can turn it back on, and
  Keyflex says so.
- **The settings file** lives in the package's private copy of `%APPDATA%`.
  Uninstalling removes it.

## Before submitting

The packaged build has to be tried at least once on a real machine, because
the startup switch cannot be tested any other way. With Developer Mode on
(Settings > System > For developers), a built layout can be installed without
signing:

```powershell
Add-AppxPackage -Register packaging\out\layout\AppxManifest.xml
```

Check that Keyflex starts from the Start menu, that tips appear, and that the
"Start when I sign in" switch survives a sign-out. Remove it afterwards with:

```powershell
Get-AppxPackage *Keyflex* | Remove-AppxPackage
```
