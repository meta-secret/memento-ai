schema Metadata:
    name: str
    labels: Labels

schema Labels:
    type: str
    location: str


projectName = "nervo-server"
volumeName = "${projectName}-vol"
appPath = "/app/nervoset"