# xargs

## Usage
```bash
xargs [options] [command [initial-arguments]]

xargs -V

```

## About
Build and execute command lines from standard input.

## Arguments
- **-0**, **--null** 
                  
    Items are separated by a null, not white space;disables quote and backslash processing and logical EOF processing.
                                           
- **-a**, **--arg-file=FILE**
          
    Read arguments from FILE, not standard input.

- **-d**, **--delimiter=CHARACTER**
 
    Items in input stream are separated by CHARACTER,not by white space; disables quote and backslash processing and logical EOF processing.
                               
                               
- **-E END**         
              
    Set logical EOF string; if END occurs as a line of input, the rest of the input is ignored (ignored if -0 or -d was specified).
                                                            
- **-e**, **--e of[=END]**           
   
    Equivalent to -E END if END is specified; otherwise, there is no end-of-file string.
                               
- **-I R**        
                 
    Same as --replace=R.

- **-i**, **--replace[=R]**        
    
    Replace R in INITIAL-ARGS with names read from standard input, split at newlines; if R is unspecified, assume {}.
                                                         
- **-L**, **--max-lines=MAX-LINES**    

   Use at most MAX-LINES non-blank input lines per command line.
                               
- **-l[MAX-LINES]**                

    similar to -L but defaults to at most one non-blank input line if MAX-LINES is not specified.
                               
- **-n**, **--max-args=MAX-ARGS**      
    
    Use at most MAX-ARGS arguments per command line.

- **-o**, **--open-tty**               

    Reopen stdin as /dev/tty in the child process before executing the command; useful to run an interactive application.
                                                              
- **-P**, **--max-procs=MAX-PROCS**

    run at most MAX-PROCS processes at a time.

- **-p**, **--interactive**
    
    Prompt before running commands.

- **--process-slot-var=VAR**   

    Set environment variable VAR in child processes.

- **-r**, **--no-run-if-empty**   
    
    If there are no arguments, then do not run COMMAND;if this option is not given, COMMAND will be run at least once.
                               
                               
- **-s**, **--max-chars=MAX-CHARS**  
  
    Limit length of command line to MAX-CHARS.

- **--show-limits**    

    show limits on command-line length.    
    
- **-t**, **--verbose**             
   
    print commands before executing them.

- **-x**, **--exit**                  

    Exit if the size (see -s) is exceeded.

    - **--help**        
           
        display this help and exit.

    _ **--version**        
        
        output version information and exit.
