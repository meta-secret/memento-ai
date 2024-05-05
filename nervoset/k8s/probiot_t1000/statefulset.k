import .common
import .pvc

apiVersion = "apps/v1"
kind = "StatefulSet"

metadata = {
    name = common.projectName
}

spec = {
    replicas = 1
    serviceName = common.projectName
    selector = {
        matchLabels = {
            app = common.projectName
        }
    }
    template = {
        metadata = {
            labels = {
                app = common.projectName
            }
        }
        spec = {
            imagePullSecrets = [
                {
                    name = "docker-registry-secret"
                }
            ]
            containers = [
                {
                    name = common.projectName
                    image = "cypherkitty/nervoset:probiot_t1000_v0.1"
                    imagePullPolicy = "Always"
                    volumeMounts = [
                        {
                            name = common.volumeName
                            mountPath = "${common.appPath}/data"
                        }
                        {
                            name = "config"
                            mountPath = "${common.appPath}/config.yaml"
                            subPath = "config.yaml"
                        }
                    ]
                }
            ]
            volumes = [
                {
                    name = common.volumeName
                    persistentVolumeClaim = {
                        claimName = pvc.metadata.name
                    }
                }
                {
                    name = "config"
                    configMap = {
                        name = "nervobot-config"
                    }
                }
            ]
        }
    }
}
