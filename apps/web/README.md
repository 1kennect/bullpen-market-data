# apps/web

Public landing site. Vite, React, TypeScript, Tailwind 4. Self-hosted fonts. Runtime dependencies: React, React Router, motion and lucide-react.

```bash
just web          # dev server on :5173
just check-web    # eslint, tsc, vitest, build
```

Set `VITE_REPO_URL` (for example `https://github.com/you/bullpen`) at build time so the site links to your repository.

The build is static with no runtime engine dependency. Deployment: see [deploy/README.md](../../deploy/README.md).
