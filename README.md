# lsearch
lsearch is a file search engine to help you quickly list and locate certain files on your system.
```
# Look for files with 'hw' in the path, `.tex` extensions and then rank by the number of 'biology' found.
lsearch --path ~/academic --content-path --has hw --content-ext --is tex --content-text --more biology
```
List all files in directory:
```
lsearch dir
```
Quickly filter files:
```
lsearch -Ee rs # file extension is rs
```
Quickly search files
```
lsearch -th ContentLoader # file has ContentLoader
```
Quickly create compound actions:
```
lsearch -th ContentLoader -Ee rs # file has content loader and file extension is rs
```
# Building
To build, simply run:
```
cargo build
```

# Installing
To install lsearch, run:
```
cargo install --path /usr/bin
```
And to fully integrate lsearch into your workflow, you can then run:
```
echo "alias ls=lsearch" >> .bashrc
```
# Usage
In lsearch, commands are simple: you tell it what content to use and what to look for:
```
--content-type --[scorer|filter] criteria
```
You use scorers to sort, and filters to refine.
For the following, the below will be used:
```
# /home/jackson/testfile.txt
Hello there!
```
## Content Types
There are several types of content. Listed are some below:

|Content Type|Content|
|---|---|
|--content-path, -P|/home/jackson/testfile.txt|
|--content-title, -T|testfile.txt|
|--content-ext, -E|txt|
|--content-text, -t|Hello there!|
|--content-exec <command>|Result of `command content-title` is content|
|--context-exif|[planned]|

## Using Content-exec
You may be thinking to yourself "Oh yay, I can search by file contents and path, but what about something like the owner?" That's a case content-exec addresses!
```
# List all files where owner is jackson
lsearch -C "stat --printf=%U" --is alerik
```
In the above command `-C` is an abreviation of `--content-exec`. We pass the command `stat --printf=%U"` command as the argument. Then, for a file `query-file` in a query, is runs `start --printf=%U $query-file` and returns the file user.

Or, we could do something else and filter our query by file permissions:
```
# List all files where permissions are like 7xx x7x xx7
lsearch -C "stat --printf=%a" --has 7
```
Similarly, in the above command `%a` denotes file permissions to `stat`.
This should prove a powerful search tool in conjunction with other system programs
## Content Scorers
Below are the content scorers in lsearch:

|Scorer|Definition|
|---|---|
|--more, -m [arg]|sum(1 for [arg] in content)|

## Content Filters
Below are some content filters:

|Filter|Definition|
|---|--|
|--is, -e [arg]|content == [arg]|
|--not, -n [arg]|content != [arg]|
|--has, -h [arg]| [arg] in content |
|--hasnt, -H [arg]| [arg] not in content|

