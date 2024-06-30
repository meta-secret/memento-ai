import taskomatic as t

img_name = lambda user: str, infra: str, registry: str, repository: str, k8s_app_name: str, app_version: str -> str {
    base_mage_name = "${registry}/${repository}:${k8s_app_name}"

    result = ""
    if infra == "prod":
        result = "${base_mage_name}_v${app_version}"
    else:
        result = "${base_mage_name}_${user}_dev_v${app_version}"

    result
}

_img_name = img_name("bynull", "prod" ,"cypherkitty", "nervoset", "groot", "0.1")

t.Taskfile {
    version = "3",
    tasks = {
        "cfg": t.Task {
           cmds = [
               'echo task-o-matic works!!! Img name: ${_img_name}',
               'echo task-o-matic works!!! Img name: ${_img_name}'
           ]
        },

        "default": t.Task {
            cmds: ["task --list-all"]
        }
    }
}

