Handling local secrets and keystore files

This repository contains developer-local sample files that may be sensitive, such as:
- `keystore.json`
- `priv.hex`
- `pwd.txt`

Current status
--------------
- These filenames are present in `.gitignore` and are not tracked in the current repository.

Recommended actions
-------------------
1) Confirm these files were never added to the repository history. If they were accidentally committed previously, consider a history rewrite (interactive rebase or `git filter-repo`) followed by force-push and coordinated secret rotation. Be careful: rewriting history affects forks and PRs.

2) If the files were only present locally (not committed), keep them out of the repository. Store them in a secure secret manager (e.g., HashiCorp Vault, cloud provider secrets manager) or a local encrypted store.

3) If any of the listed files may have been leaked, rotate the secrets immediately: generate new keys, revoke old ones, and update any consumers.

4) Avoid embedding secrets in CI logs or artifacts. Ensure CI workflows do not `echo` sensitive data and that artifact upload steps do not include secret files.

5) Add a short `README` note in relevant crates documenting how to generate test vectors if needed, rather than storing real secrets in the repo.

If you want, I can:
- Search the git history for any accidental commits of these files and prepare a `git filter-repo` plan to remove them.
- Create a short script to safely move local secret files into a `secrets/` directory and encrypt them with `gpg` for local storage (not committed).
