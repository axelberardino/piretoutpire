# Limitations

## Id limitation

As the id used is an uint32, only 4 billions users could theorically joins. As
such, only 4 billions differents files or message could be shared.

## Id hash collision

To keep thing simple, I used a simple crc as a poor-man hash. It's (somewhat)
easy to get collisions which could affect the proper behavior of this system.
For demo and test purpose, it should be fine, though.

## Scaling

As it's using the Kademlia DHT algorithm, it should scale fairly well. In fact,
it should even function better with a lot of users. The routing table size is
limited, which should prevent individual scaling issue.

One weakness, though, would be about the stored values, and the files<->peers
mapping. As it is unbounded, it can grow indefinitively.

## Scarcity

Not having a lot of peers could be an issue. Few active peers means a lot of
"hole" in the global peers graph. For example, having two group of graph losing
their only link, means some peer clusters can't see each other. It could be
mitigated by spawning some peers as persistant trackers, and let new peers
entering the network through them.

## File sharing

Only files can be shared. You can't share folders. If you want to share a
folder, you'll have to share each files individually, and lose the tree
arborescence (which is quite annoying).

Also, as u32 is used everywhere, files has the same kind of limitations: no file
bigger than 4 Go, can be shared.

## Honesty/correctness

The whole system relies on the fact peers will tell the truth, and don't
contradict each other. There's no mecanism to ensure the correctness of
information shared between peers.

Even non malicous peer could compromise the system by giving out-dated or
corrupted information (for example, not using the same chunk size, or using an
already used peer id).

## Network

The network implementation is fairly naive. There's no exponential-backoff
retry, or even simple retry!

"Bad" nodes are forgotten between binary launch. So they're queried again.

Having a bunch of churn peers in our DHT would mean we will need some time to
get back active peers (because peers are deemed bad only after a few retry).
