### Nervoset build

#### Arhitecture (universal way of working with nervoset build and automation system):
 - there is only one central entry point to build everything in nervoset, it's called taskfile. 
 - your investigation journey starts with going to "infra" or "nervoset" directory, running `devbox shell` and
   then typing `task` command, which shows all possible tasks we have in nervoset
 - to find a particular task, you can just `juess and grep` by typing, for instance: `task | grep config` or `task | grep deploy`
