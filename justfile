set windows-shell := ["pwsh.exe", "-c"]

build:
  npm run prepublishOnly

link:
  pnpm link --global