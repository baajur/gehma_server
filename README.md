## Testing

Throttle connections:

```
tc qdisc add dev wlo1 root netem delay 2s
```

Reset

```
sudo tc qdisc del dev wlo1 root
```
