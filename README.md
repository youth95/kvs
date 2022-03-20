# [WIP] KVS

Key Value Services is a cli tools that help to you create your Key-Value store services.

# Motivation

I want to create a content sharing tool that is managed by only one person and readable by many people in a secure network environment, so that I can use some common content in the terminal.


# Install

```bash
cargo install key_value_service
```

# Usage

1. Start kvs in your server

```bash
> kvs start
2022-03-18T15:59:26.503861Z  INFO starting with 0.0.0.0:8888 successfully!
```

2. Login the kvs services from client
```bash
> kvs -r 0.0.0.0:8888 login
2022-03-18T16:05:05.022305Z  INFO Token: FAAAAAAAAACUH40t6d+A9jzhexGHktUINvWwL317xp1/AQAAFAAAAAAAAACrtlLcjeSqhMZCj5rnNn2hkf0K/w==
2022-03-18T16:05:05.022393Z  INFO Save Token file to: .kvs/token
```


3. Create a private key value
```
> kvs create foo "hello world"
```

default, kvs will encrypt the value use your `priv_key` in local. Remote just judge the key's owner. The decryption process needs to be completed by the client itself.

4. Read a key
```
> kvs read foo
hello world
```

5. Create a public key value

```
> kvs create priv_foo "priv hello world" -p
```

If you just do. kvs will send the value and save value as plaintext in remote.


6. Read a private key
```
> kvs cat priv_foo
priv hello world
```

7. Delete a Key
```
> kvs delete priv_foo
```

You just can delete key that owner is you.

8. Update a Key
```
kvs create priv_foo "this is change data"
kvs read priv_foo
this is change data
```

# Example

Your team have a big list of resource id.

```bash
# important_resource_id.txt
...
A7EB0B7B-3E27-4531-B239-750300EE8D0C
B9164246-49E3-45F1-A8CE-D871CEBE3971
92312D9B-B2D8-4A92-B7B5-9A5491FF9BEC
8E1B24ED-0023-45E0-BF34-1548C8D8884D
...
```

Some times, the team member need create the url by the big list.

```bash
# important_resource_urls.txt
...
http://xxx.xxx.xxx.xxx/app/goods/A7EB0B7B-3E27-4531-B239-750300EE8D0C
http://xxx.xxx.xxx.xxx/app/goods/B9164246-49E3-45F1-A8CE-D871CEBE3971
http://xxx.xxx.xxx.xxx/app/goods/92312D9B-B2D8-4A92-B7B5-9A5491FF9BEC
http://xxx.xxx.xxx.xxx/app/goods/8E1B24ED-0023-45E0-BF34-1548C8D8884D
...
```

Let's assume we have written a command line tool named `transform` to handle this case.

```
cat important_resource_urls.txt | transform
```

The question is, how do I share `important_resource_urls.txt` with other team members.

Granted, there are many ways to share it. But you can fast finish it if you use the `kvs`.

```bash
# create the key
kvs create important_resource_urls -f important_resource_urls.txt

# and then, other team member can use it.
kvs cat important_resource_urls | transform
```

# Road Map
* [*] add `set` and `get` command to config some value in client local
* [ ] add `upload` command to upload all file in current directory and use the relative directory as key.
* [ ] add `set` command to set the config in client local.
