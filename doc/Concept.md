# How DHT and file sharing work

Sharing messages and files through a decentralized network is not an easy task.
Peer to peer works with a way to link peers together. One of the way to achieve
that, is thought a distributed hash table.

One possible implementation, would be to have a hashmap on each peer, and
everytime someone is joining or quitting the network, we update this table for
everybody. But it's not very scalable. Another way is to maintain only a small
subpart of a global hash map, and if there are enough people to keep updating
their parts, the hash map should be up-to-date.

# Peer search strategy

## Query all

Simply query all the peers we already know, until we found what we're searching
for. It's pretty effective on very small networks. But on network with just a
hundreds of users, it becomes quickly irrelevant.

## Spreading

Starting from a given peer, just ask all peers of peers. A round of getting a
peer's peers is a hop. One can defined a not too big maximum numbers of hop (but
the search stop if the resource is found). It's working, even for a big network
(if the hop is not too high), but it consumes a lot of bandwith. As most queries
will not be successful, it's quite a waste.

## Closest peers

A better strategy would be to only queries few peers, and still be sure to get
the answer (or getting close to it). To achieve that, we need to attribute peers
a number, and let "close" peers announce knows each other. By doing so, we can
now search until we're not closer to the result. Because the data is ordered by
closeness, we can effectively know ahead if a path is the right one, without
fully visiting it. We guaranted to visit O(log(n)) peers before stopping.
It's pretty efficient for big network, but come with some downside for small
ones (scarcity is an issue).

# File or message sharing

From going on, we will assume the closest peers strategy.

Sharing a file or a message is very similar to finding a peer. Peers and
files/messages share the same id range. So, peers close to the file/message id,
is responsible to maintain it. Usually many peers are maintaining a single
file/message.

Finding a file/message is very close to the peer search. Simply find the closest
peers from the file/message id, and then ask them if they store the associated
value.
