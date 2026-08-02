# LAZIN: LAZy INterlinker

Primarily a command line utility for managing dotfiles.

## Usage

Initialize an example configuration using `lazin init`
or create a directory which will store the configuration files.
All files ending in `.toml` will be loaded as configuration.

Configuration consists of two distinct structures; `modules` and `workspaces`.
A `module` defines source files/directories that can be linked to a desired target,
while a `workspace` defines a collection of `modules` that will be linked.

Example configuration:

```toml
my_workspace = ["my_module"]

[my_module]
# shorthand definition
source_file = "/target/file"
# longer definition with optional `config`, see example below
another_file = {
    path = "/another/target/file"
}

[my_module_with_encryption]
# `config` is a reserved keyword
# can be used at module level and is used as default
# for all configured paths
config = { encrypt = true, recipient = "some-gpg-recipient" }
unencrypted_directory = {
    path = "/target/encrypted_directory"
    config = { encrypt = false }
}
encrypted_source_file = {
    path = "/encrypted/target/file"
    # Encryption and recipient can be defined per module entry 
    # and override the module level configured value
    config = { encrypt = true, recipient = "another-gpg-recipient" }
}
```

After writing the configuration it can be validated using `lazin check` or
by running `lazin link <WORKSPACE_NAME>`, calling `link` automatically validates
the config and reports any errors. By default linking is done as a `dry run`
to link it is required to provide a `--link` or `-l` flag.

`lazin encryption` can be used to encrypt files using `gpg` and a `recipient`.
For this to work `gpg` needs to be installed your computer.
Using `lazin encrypt` any source files will be added to `.gitignore` under a managed
section, and for each encrypted file a file ending in `.gpg` will be created.
To decrypt files use `lazin encrypt -r`.

## Similar tools

Great tools that are battle tested and work much better:

- [GNU Stow](https://www.gnu.org/software/stow/)
- [Dotter](https://github.com/SuperCuber/dotter)

## TODO

- [x] Add force option when linking to override existing files
- [x] Add option to skip linking files which couldn't be decrypted
- [x] Add option to specify .gitignore path
- [x] ~~Add logic for finding the nearest .git directory instead of assuming
command is ran in the correct directory~~
Won't do: no clear path where to stop the search
- [ ] Add logic for removing links from previously linked workspaces when linking
another workspace
- [ ] Thinkup a better solution than reserving `config` as a keyword in modules
- [ ] Allow linking to directories requiring a privileged user
- [x] Add message when running as sudo
- [ ] Update gpg wrapper to allow using passphrases in keys
