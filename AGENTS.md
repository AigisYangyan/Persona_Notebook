# Launch Entry Policy

This project has one canonical launch directory:

`启动入口/`

Rules for future maintenance:

1. All user-facing launch files must live in `启动入口/`.
2. Root-level launch scripts are compatibility wrappers only. They must delegate into `启动入口/` and must not contain independent launch logic.
3. When changing how the desktop app is built or opened, update files in `启动入口/` first.
4. Do not introduce new launch paths outside `启动入口/` unless the user explicitly asks for them.
5. If launch behavior changes, update `启动入口/README.txt` in the same change.
