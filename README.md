# lsearch
lsearch is a file search engine. Think of it like a Google (or DuckDuckGo) for files on your own PC! And unlike Google, you can figure it like you want.
```
# Look for files with 'hw' in the path, `.tex` extensions and then rank by the number of 'biology' found.
lsearch --path ~/academic --content-path --has hw --content-ext --is tex --content-text --more biology
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
|--content-path|/home/jackson/testfile.txt|
|--content-title|testfile|
|--content-ext|txt|
|--content-text|Hello there!|
|--context-exif|[planned]|
|--content-exec|[planned]|

## Content Scorers
Below are the content scorers in lsearch:

|Scorer|Definition|
|---|---|
|--more [arg]|sum(1 for [arg] in content)|

## Content Filters
Below are some content filters:

|Filter|Definition|
|---|--|
|--is [arg]|content == [arg]|
|--not [arg]|content != [arg]|
|--has [arg]| [arg] in content |
|--hasnt [arg]| [arg] not in content|

