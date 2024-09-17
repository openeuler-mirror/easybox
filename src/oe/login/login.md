# taskset

## Usage
```
login [-p] [-h <host>] [-H] [[-f] <username>]
```

## About

Begin a session on the system.

## Description

**login** is used when signing onto a system. If no argument is given, **login** prompts for the username.

The user is then prompted for a password, where appropriate. Echoing is disabled to prevent revealing the password. Only a number of password failures are permitted before **login** exits and the communications link is severed. See **LOGIN_RETRIES** in the config file items.

If password aging has been enabled for the account, the user may be prompted for a new password before proceeding. In such case old password must be provided and the new password entered before continuing.

The user and group ID will be set according to their values in the /etc/passwd file. There is one exception if the user ID is zero. In this case, only the primary group ID of the account is set. This should allow the system administrator to login even in case of network problems. The environment variable values for **\$HOME, \$USER, \$SHELL, \$PATH, \$LOGNAME**, and **\$MAIL** are set according to the appropriate fields in the password entry. $PATH defaults to /usr/local/bin:/bin:/usr/bin for normal users, and to /usr/local/sbin:/usr/local/bin:/sbin:/bin:/usr/sbin:/usr/bin for root, if not otherwise configured.

The environment variable **$TERM** will be preserved, if it exists, else it will be initialized to the terminal type on your tty. Other environment variables are preserved if the **-p** option is given.

The environment variables defined by PAM are always preserved.

Then the user’s shell is started. If no shell is specified for the user in /etc/passwd, then /bin/sh is used. If the specified shell contains a space, it is treated as a shell script. If there is no home directory specified in /etc/passwd, then / is used, followed by .hushlogin check as described below.

If the file .hushlogin exists, then a "quiet" login is performed. This disables the checking of mail and the printing of the last login time and message of the day. Otherwise, if /var/log/lastlog exists, the last login time is printed, and the current login is recorded.

## Options

**-p**
<br />
&emsp;&emsp;Used by **getty** to tell **login** to preserve the environment.

**-f**
<br />
&emsp;&emsp;Used to skip a login authentication. This option is usually used by the **getty** autologin feature.

**-h**
<br />
&emsp;&emsp;Used by other servers (such as **telnetd**) to pass the name of the remote host to **login** so that it can be placed in utmp and wtmp. Only the superuser is allowed use this option.

&emsp;&emsp;Note that the **-h** option has an impact on the PAM service name. The standard service name is login, but with the -h option, the name is remote. It is necessary to create proper PAM config files (for example, /etc/pam.d/login and /etc/pam.d/remote).

**-H**
<br />
&emsp;&emsp;Used by other servers (for example, telnetd(8)) to tell login that printing the hostname should be suppressed in the login: prompt. See also LOGIN_PLAIN_PROMPT below.

**--help**
<br />
&emsp;&emsp;Display help text and exit.

**-V, --version**
<br />
&emsp;&emsp;Print version and exit.
