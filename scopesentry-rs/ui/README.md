Scopesentry RS UI (React + Shadcn + Bun)

- Bun-only environment (no Node). Commands use `bun`.
- Vite + React + TypeScript + TailwindCSS + shadcn/ui.

Scripts
- `bun dev`: start dev server
- `bun build`: production build
- `bun preview`: preview build

Configure
- API base URL via `src/lib/config.ts` or `VITE_API_BASE` env.
- Scheduler default: http://localhost:8083

Features
- Dashboard with quick stats placeholder
- Nodes online list (GET /api/node/data/online)
- Task creation form (POST /api/task/add)

Folder structure
- `src/pages`: route pages
- `src/components/ui`: shadcn-style primitives
- `src/lib`: api, config, utils
