schema Metadata:
    name: str
    labels: Labels

schema Labels:
    type: str
    location: str


projectName = "kevin"
volumeName = "${projectName}-vol"
appPath = "/nervoset/app/jarvis"
agentType = "kevin"