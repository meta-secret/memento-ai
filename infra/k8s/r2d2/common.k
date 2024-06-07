schema Metadata:
    name: str
    labels: Labels

schema Labels:
    type: str
    location: str


projectName = "r2d2"
volumeName = "${projectName}-vol"
appPath = "/app/nervoset"