# Study Session Records

This directory stores generated markdown records for local manual `rpdf` study-session runs.

Default git behavior:

- generated `study_sessions/*.md` files are ignored by `.gitignore`
- `README.md` stays tracked
- if a specific session record contains durable project findings, it can still be committed deliberately with `git add -f`

Use:

```bash
./scripts/run_study_session_checklist.sh <readable-pdf> <weak-pdf> <image>
```

The script:

- checks the supplied local assets
- reruns the automated baseline
- records whether a graphical display is available
- creates a timestamped session markdown file in this directory

Commit generated session records only if they contain project-relevant validation findings that should live in the repo.
