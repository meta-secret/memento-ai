variable "repo_name" {
    default = "nervo_bot"
}

resource "null_resource" "provision" {
    triggers = {
        //file_exists = fileexists("$HOME/prod/${var.repo_name}")
        always_trigger = timestamp()
    }

    provisioner "local-exec" {
        command = <<-EOF
            #gh auth login --with-token < tmp/github_token.txt
            git clone https://$(cat tmp/github_token.txt)@github.com/cypherkitty/nervo_bot.git "$HOME/prod/${var.repo_name}" || true
        EOF
    }
}