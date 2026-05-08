# Study Session Records

This directory stores generated markdown records for local manual `rpdf` study-session runs.

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
