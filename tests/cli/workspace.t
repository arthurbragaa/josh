  $ export TESTTMP=${PWD}
  $ git init -q repo
  $ cd repo
  $ mkdir -p apps/frontend libs/shared
  $ echo app > apps/frontend/app.txt
  $ echo shared > libs/shared/shared.txt
  $ git add .
  $ git commit -q -m initial

Create a workspace from repository paths.

  $ josh workspace create workspaces/frontend --map app=apps/frontend \
  >     --map shared=libs/shared
  Created workspace 'workspaces/frontend'
  Definition: workspaces/frontend/workspace.josh

  $ cat workspaces/frontend/workspace.josh
  app = :/apps/frontend
  shared = :/libs/shared

Add a path to the workspace from inside it.

  $ cd workspaces/frontend
  $ josh workspace add libs/shared --as libs/extra
  Mapped 'libs/shared' to 'libs/extra'
  Definition: workspaces/frontend/workspace.josh
  $ tail -n 1 workspace.josh
  libs/extra = :/libs/shared
  $ cd ../..

Add initializes the current workspace when needed.

  $ mkdir workspaces/current
  $ cd workspaces/current
  $ josh workspace add libs/shared --as shared
  Mapped 'libs/shared' to 'shared'
  Definition: workspaces/current/workspace.josh
  $ cat workspace.josh
  shared = :/libs/shared
  $ cd ../..

Existing definitions are protected.

  $ josh workspace create workspaces/frontend 2>&1
  Error: Workspace 'workspaces/frontend' already exists
  Workspace 'workspaces/frontend' already exists
  [1]

Dry-run validates without writing.

  $ josh workspace create workspaces/backend --map app=apps/frontend --dry-run
  Would create workspace 'workspaces/backend'
  Definition: workspaces/backend/workspace.josh
  app = :/apps/frontend
  $ test ! -e workspaces/backend

Invalid mappings do not create a workspace.

  $ josh workspace create workspaces/invalid --map app=../frontend >/dev/null 2>&1
  [1]
  $ test ! -e workspaces/invalid
