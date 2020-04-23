# Gehma [![Build Status](https://travis-ci.org/kper/gehma_server.svg?branch=master)](https://travis-ci.org/kper/gehma_server)

Die Gehma-App versucht das leidvolle Suchen von Freunden für Events zu verbessern. Denn motivierte Leute suchen nicht mehr, sondern finden. Ist nichts dabei was einem selbst gefällt, kann man auch selber Vorschläge machen. (TODO copy english)


## Testing

Throttle connections:

```
tc qdisc add dev wlo1 root netem delay 2s
```

Reset

```
sudo tc qdisc del dev wlo1 root
```
