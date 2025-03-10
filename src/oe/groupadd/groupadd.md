# groupadd

## Usage
```bash
groupadd [options] GROUP
```

## About
Add the group to the system.

## Arguments
- **-f, --force**

    exit successfully if the group already exists,and cancel -g if the GID is already used
- **-g, --gid GID**

    use GID for the new group
- **-h, --help**
-
    display this help message and exit
- **-K, --key KEY=VALUE**

    override /etc/login.defs defaults
- **-o, --non-unique**

    allow to create groups with duplicate(non-unique) GID
- **-p, --password PASSWORD**

    use this encrypted password for the new group
- **-r, --system**

    create a system account
- **-R, --root CHROOT_DIR**

    directory to chroot into
- **-P, --prefix PREFIX_DIR**
    directory prefix
- **-U, --users USERS**

    list of user members of this group
