### Just hacks

- setup just autocomplete for bash (for zsh, replace `bash` -> `zsh`, `~/.bashrc` -> `~/.zshrc`):
  - mkdir -p ~/.just
  - just --completions bash > ~/.just/just-completion.bash
  - echo "source ~/.just/just-completion.bash" >> ~/.bashrc
  - source ~/.just/just-completion.bash