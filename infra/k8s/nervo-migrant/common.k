schema Metadata:
    name: str
    labels: Labels

schema Labels:
    type: str
    location: str


projectName = "nervo-migrant"
volumeName = "${projectName}-vol"
appPath = "/nervoset/app/nervo-migrant"
