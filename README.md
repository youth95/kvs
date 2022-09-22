# KVS

Key Value Services is a cli tools that help to you create your Key-Value store services.



# Motivation

I want to create a content sharing tool that is managed by only one person and readable by many people in a secure network environment, so that I can use some common content in the terminal.


# Install

```bash
cargo install key_value_service
```

# Warning

This project is still in the early stage of development and is only used as a development tool. Do not use it for data storage in the production environment. If you have this requirement, I recommend that you use [redis](https://github.com/redis/redis). Some APIs will undergo destructive changes without notice.

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

9. show remote info
```
> kvs remote
0.1.3
```

10. show local info
```
> kvs local
scope: 0x4d7153428dd617a410f114468d212a9cd1b7ccd0
pub: MIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQD9qWAweIhnLfBdjYj8oty1z2FYycQ8qhebrDLCQBJPTF1IfV282WCHET7Fsjd1C9+XMbW2xT0f73cZgSExgILeGjZFlx9gEf5VVJyBezfQ6yU2V9Emo58zRh6fjfoBGUsXmVIJIGCpnVjHg/ECEVKuDVQ3h0SEHrdxE98bSl9RIQIDAQAB
```

11. Read other scope key
```
> kvs read -s 0xad359ae3e478342ed2b5512ed7ff4ebb3ceb2dd test_pub
pub content
```

12. Restart the kvs Server 
```
> kvs restart
```

13. Stop the kvs Server
```
> kvs stop
```

14. remove all keys
```
> kvs list | awk -F '\t' '{print "kvs delete " $2}' | bash
```
# Examples

## Case1 sync info in one team
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
kvs read important_resource_urls | transform
```

## Case1 sync info in public

```bash
curl https://xxx/a.json | json "important" | kvs create important -f -p
kvs read important -s your_scope
```

## Case2 prove yourself

You can tail your `pub key` to some website.

```
> kvs local
scope: 0x4d7153428dd617a410f114468d212a9cd1b7ccd0
pub: MIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQD9qWAweIhnLfBdjYj8oty1z2FYycQ8qhebrDLCQBJPTF1IfV282WCHET7Fsjd1C9+XMbW2xT0f73cZgSExgILeGjZFlx9gEf5VVJyBezfQ6yU2V9Emo58zRh6fjfoBGUsXmVIJIGCpnVjHg/ECEVKuDVQ3h0SEHrdxE98bSl9RIQIDAQAB
```

Website will tail you a cipher text.

```
website: (your pub key) + (random value) -> (cipher text)
```

Now you can use `kvs de` command to decrypt the (cipher text)

```
> kvs de (cipher text)
(random value)
```

Tail the (random value) to website. you will prove yourself to website.

# Road Map

Community
* [x] add `sync` command to sync all file in one directory and use the relative directory as key.
* [x] add `list` command to list all key meta in your scope.
* [x] add `restart` command to restart the server.
* [x] add `stop` command to stop the server.
* [x] add `set` and `get` command to config some value in client local.
* [x] add `set` command to set the config in client local.
* [x] add `--file` option in create and upload command.
* [x] add the same option in `sync` command look like `create` `update`.
* [x] append show public key in `kvs local`.
* [x] add `kvs de` command to decrypt some content. use local private key by default.
* [x] add `kvs en` command to encrypt some content. use local public key by default.

* [ ] remove `--scope` option in read, you can use `kvs read your_scope:some_key` to read a public key.
* [ ] add `kvs set whitelist` command to set a whitelist.
* [ ] add `kvs search` command to search some content in different repository.



* [x] fix `--file` option in create and upload command can be not give the filename, kvs will use the stdin content as value if you do that.
* [ ] add server config to config the store backend.
* [ ] add unit test and docs.
* [ ] ~~add github action to release the bin file.~~
* [ ] refactor the `Remote Action` model.
* [ ] refactor the `aes session` to be a `aes stream`.
* [ ] config docker container.


* [ ] ``

Commercial
* [ ] add the p2p in share key progress.
* [ ] Build a free central storage node.
* [ ] add `upgrade` command to sync the remote `kvs` cli to client local.

