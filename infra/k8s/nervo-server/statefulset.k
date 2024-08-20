import .common

apiVersion = "apps/v1"
kind = "StatefulSet"

metadata = {
    name = common.projectName
    labels = {
        app = common.projectName
    }
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
                    imagePullPolicy = "Always"
                    image = option("imageName")
                    command = ["/bin/sh", "-c", "./nervo_server"]
                    ports = [
                        {
                            containerPort = 3000
                        }
                    ]
                    volumeMounts = [
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
                    name = "config"
                    configMap = {
                         name = "${common.projectName}-config"
                    }
                }
            ]
        }
    }
}
