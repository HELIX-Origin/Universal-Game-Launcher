# 🐛 Confirmed bugs

The issues below are confirmed by inspecting the current source. They are **not** reports of behavior reproduced on every operating system. Add environment details when a runtime issue is reproduced, and remove an entry only after fixing and verifying it.

## ✅ Resolved in source

The starter screen described below has been replaced by a library interface. Source inspection and successful frontend type/build checks confirm it no longer calls the unregistered `greet` command or references the absent starter logos. Desktop runtime behavior has not yet been smoke-tested.

### B-01 — Starter page called a command that was not registered

- **Resolution:** Replaced the starter form with the library view. The current page invokes registered library and game-action commands and displays failures.

### B-02 — Starter page requested logo files that were missing

- **Resolution:** Removed the starter links and logo requests when replacing the page.

## 📝 Reporting another issue

Include the operating system and relevant client versions, exact reproduction steps, expected and actual behavior, the source area involved, and whether the issue was reproduced or inferred from code. Never include API keys, credentials, private machine paths, or personal game-library data.
