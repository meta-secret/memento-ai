import .common

apiVersion = "v1"
kind = "PersistentVolumeClaim"
metadata = {
    name = "${common.projectName}-pvc"
}

spec = {
    accessModes = [
        "ReadWriteOnce"
    ]
    storageClassName = "local-path"
    resources = {
        requests = {
            storage = "16Gi"
        }
    }
}
