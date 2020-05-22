# Postfix Queue Management

Dump all messages currently in the queue in JSON format:

```
postqueue -j
```

Delete all enqueued messages:

```
postsuper -d ALL
```
