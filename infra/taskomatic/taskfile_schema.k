schema Taskfile:
    version: str
    tasks: {str: Task}

schema Task:
    #deps: [str]
    cmds: [str]
