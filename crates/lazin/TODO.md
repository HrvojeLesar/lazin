Lazin - Dotfile manager
===============

Expected features
---------------

- [ ] Multiple configuration
    - [x] Define dotfiles to copy for each configuration
    - [x] Define naming
        - `Workspace`: A named collection of `Module`s to be
        written to config folders
        - `Module`: A named collecation of `Config pairs`
        - `Config pairs`: Key value pairs. Key defines the source file,
        while value defines the path to which the file should be copied to
        - `Attributes`: Will most likely require custom config file
        or just a sub attribute in module definition.
        Defines custom properties for a desired module,
        or a single module pair
- [x] Encryption/Decryption
    - [x] Allow users to encrypt desired files
    - [ ] Check git or cache a hash of the original file
    - [x] Automatic `.gitignore` management for encrypted source
    files (only if git repository is detected)
- [ ] Custom config language, toml like with @Attributes
