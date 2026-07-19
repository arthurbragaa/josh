  $ export TESTTMP=${PWD}


  $ cd ${TESTTMP}
  $ mkdir remote
  $ cd remote
  $ git init -q libs 1> /dev/null
  $ cd libs

  $ mkdir sub1
  $ echo contents1 > sub1/file1
  $ git add sub1
  $ git commit -m "add file1" 1> /dev/null

  $ echo contents2 > sub1/file2
  $ git add sub1
  $ git commit -m "add file2" 1> /dev/null


  $ mkdir sub2
  $ echo contents1 > sub2/file3
  $ git add sub2
  $ git commit -m "add file3" 1> /dev/null
  $ git branch feature

  $ cd ${TESTTMP}

  $ which git
  /opt/git-install/bin/git

  $ josh clone remote/libs :/sub1 libs
  Added remote 'origin' with filter ':/sub1'
  From file://${TESTTMP}/remote/libs
   * [new branch]      feature    -> refs/josh/remotes/origin/feature
   * [new branch]      master     -> refs/josh/remotes/origin/master
  
  From file://${TESTTMP}/libs
   * [new branch]      feature    -> origin/feature
   * [new branch]      master     -> origin/master
  
  Fetched from remote: origin
  Already on 'master'
  
  Cloned repository to: ${TESTTMP}/libs/

  $ cd libs


  $ tree .git/refs
  .git/refs
  |-- heads
  |   `-- master
  |-- josh
  |   |-- cache
  |   |   `-- 29
  |   |       `-- 0
  |   |           `-- bf567e0faf634a663d6cef48145a035e1974ab1d
  |   |-- filtered
  |   |   `-- bf567e0faf634a663d6cef48145a035e1974ab1d
  |   |       `-- heads
  |   |           `-- master
  |   `-- remotes
  |       `-- origin
  |           |-- feature
  |           `-- master
  |-- namespaces
  |   `-- josh-origin
  |       |-- HEAD
  |       `-- refs
  |           `-- heads
  |               |-- feature
  |               `-- master
  |-- remotes
  |   `-- origin
  |       |-- HEAD
  |       |-- feature
  |       `-- master
  `-- tags
  
  18 directories, 11 files

  $ tree
  .
  |-- file1
  `-- file2
  
  1 directory, 2 files

  $ git checkout feature
  branch 'feature' set up to track 'origin/feature'.
  Switched to a new branch 'feature'

  $ tree
  .
  |-- file1
  `-- file2
  
  1 directory, 2 files

The path form produces the same projection and remote configuration.

  $ cd ${TESTTMP}
  $ josh clone remote/libs --path sub1 --into path-clone >/dev/null 2>&1
  $ test "$(git -C libs rev-parse master)" = "$(git -C path-clone rev-parse HEAD)"
  $ cmp libs/.git/josh/remotes/origin.josh path-clone/.git/josh/remotes/origin.josh

Clone paths are validated.

  $ josh clone remote/libs --path ../sub1 2>&1
  Error: Clone path '../sub1' must be relative and remain inside the repository
  Clone path '../sub1' must be relative and remain inside the repository
  [1]
  $ test ! -e sub1

  $ josh clone remote/libs --path . >/dev/null 2>&1
  [1]
  $ josh clone remote/libs --path /sub1 >/dev/null 2>&1
  [1]

The path and filter forms cannot be combined.

  $ josh clone remote/libs :/sub1 legacy-out --into ignored >/dev/null 2>&1
  [2]
  $ test ! -e legacy-out
  $ test ! -e ignored
