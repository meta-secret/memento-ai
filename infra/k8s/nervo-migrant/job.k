import .common

apiVersion = "batch/v1"
kind = "Job"

metadata = {
    name = common.projectName
    labels = {
        app = common.projectName
    }
}

spec = {
    template = {
        metadata = {
            labels = {
                app = common.projectName
            }
        }
        spec = {
            restartPolicy = "Never"
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
                    command = ["/bin/sh", "-c", "./nervo-migrant migration"]
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
