schema Metadata:
    name: str
    labels: Labels

schema Labels:
    type: str
    location: str


projectName = "groot"
volumeName = "${projectName}-vol"
appPath = "/app/nervoset"