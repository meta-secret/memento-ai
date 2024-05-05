
schema Metadata:
    name: str
    labels: Labels

schema Labels:
    type: str
    location: str


projectName = "probiot-t1000"
volumeName = "${projectName}-vol"
appPath = "/app/nervoset"