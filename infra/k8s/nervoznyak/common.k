schema Metadata:
    name: str
    labels: Labels

schema Labels:
    type: str
    location: str


projectName = "nervoznyak"
volumeName = "${projectName}-vol"
appPath = "/nervoset/app/jarvis"
agentType = "nervoznyak"
