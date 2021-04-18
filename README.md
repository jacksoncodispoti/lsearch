# lsearch
lsearch is a file search engine. Think of it like a Google (or DuckDuckGo) for files on your own PC! And unlike Google, you can figure it like you want.
```
# Look for files with 'hw' in the path, `.tex` extensions and then rank by the number of 'biology' found.
lsearch --path ~/academic --content-path --has hw --content-ext --is tex --content-text --more biology
```
Quickly filter files:
```
lsearch -Ee rs
``
Quickly search files
```
lsearch -th ContentLoader
```
Quickly create compound actions:
```
lsearch -th ContentLoader -Ee rs
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
|--content-title, -T|testfile|
|--content-ext, -E|txt|
|--content-text, -t|Hello there!|
|--context-exif|[planned]|
|--content-exec|[planned]|

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

