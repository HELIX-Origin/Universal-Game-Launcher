# 🛡️ Safety rules

Apply these safeguards to every task, including documentation, tests, and handoffs:

- 🎯 Keep changes within the assigned scope; avoid unrelated refactors.
- 🔒 Never commit secrets, real user-library data, or fixtures containing credentials. Metadata enrichment is optional and must remain opt-in.
- ✅ Preserve URI and executable validation, stable platform identifiers, and the separation between local scanning and official-client launch/install.
- 🧾 Do not claim an integration, translation, or feature works merely because a model or message key exists.
- 🕵️ Keep private machine paths and local application data out of bug reports, logs, and handoffs.
