# Tower Defense

# Tower_Balance

by The Balancing Tower

## Team Members
* Advanced Topic Subteam 1: Custom Physics Engine
	* ISW26: Ishay Wirthiem
 	* gmb139: Gio Bair
 	* brr154: Ben Rand

* Advanced Topic Subteam 2: Network Multiplayer
	* pin7: Polly Naneva
 	* OSO36: Osarumen Samantha Okhiku
 	* dag327: Daniel Gallagher

## Game Description

Face off against an opponent in our tower defense balance building game.
Each Player has a tower that they have to build up, defend, and gear up to attack their opponent.
Keep your tower above the fog line or you lose, you'll also lose if you opponent knocks down your tower.
That might sound easy, but your tower will also have to be balanced or it will fall over.
Each block has a mass, shape, and density that will impact the balance of your tower.

## Game Concept Photos
<img width="512" height="239" alt="TD concept photo" src="https://github.com/user-attachments/assets/318e508a-f406-440a-8915-1b5604770d95" />  
Danny
<img width="512" height="380" alt="TD concept photo2" src="https://github.com/user-attachments/assets/2cdad1c5-d81a-48c7-b689-5791cb78f466" />  
Gio

## Advanced Topic Description

### Custom Physics Engine

**Rigid-Body Simulation:** Different building block texture types with distinct mass, friction, and collision responses. 

For our physics engine we will need to calculate the velocities, accelerations, and positions for each physics supporting object in the game scene. This will be done using the Semi-Implicit Euler Integration method which states: 

V<sub>n+1</sub> = V<sub>n</sub> + hA<sub>n</sub>
X<sub>n+1</sub> = X<sub>n</sub> + hV<sub>n+1</sub>

Where h = change in time = Δt, V = velocity, X = position, A = acceleration.

We will use these equations to integrate the velocities of each object to obtain their position in the next frame. The acceleration will be initially determined only by the force of gravity, and only after interacting with other objects will a block’s acceleration change.

The same calculation will also be performed on each object's angular velocity to obtain the angular momentum at any given frame.


**Collision Detection** Using the separating axis theorem (SAT) each object will check if it is colliding with another. This theorem is used to determine if two convex shapes are intersecting. This is important for our simulation because the blocks can be rotated by the player before being dropped onto the tower below. Further, we will need to split blocks into individual cubes and re-attach to allow for the ‘convex’ requirement to hold.

SAT - "If two convex objects are not penetrating, there exists an axis for which the projection of the objects will not overlap."

Objects will need to be checked pairwise to determine if they overlap. For each object the normal vectors of its sides will be calculated, and these normals will be the axes for which we will project the objects. According to SAT, if any of these projections has a gap, then the two objects are not overlapping. If this narrow phase algorithm proves to be too slow, we may also implement a broad phase collision detection algorithm to reduce the number of collision checks per frame.

**Structure Stability:** Towers lean, wobble, and topple over based on weight distribution and block type.

**Physics Driven Attacks:** impact and knockback forces push through connected blocks for realistic, readable damage.



    
### Network Multiplayer

**Host/Client Model:** Players connect to a server instead of peer to peer. (Separate the client code from the server code) . We will have an authoritative server that deals with the actual state of the game. Clients and Server each have their own folder. Looking into a rollback system where the client does calculations locally (client side prediction), and then that is checked with the Server. If no problems good, if there are we rollback. To rollback we can compare the differences in the game instances, if too much there will be a laggy jump, otherwise we could slowly sync up the client to the server. For example, if the client had a rocket moving a little too fast, the server could send signals to make it slower until its at the right location.

**Real Time State Sync:** Everything stays synchronized between clients like block placements, currency, and attacks. We will have the game running at a set tick amount. Using a fixed update we can set the tick amount to our desired amount. To start looking at using 30hz but if that does not play well we can increase the value. (Looking at using the Tokio crate to help with staying in sync).

**Latency Handling:** Keeps game fair under lag. (We will be Using UDP over TCP. Even though UDP is less reliable its speed will make up for it). Rust has a built in UDPSocket in the standard library. We can bind that socket to a given IP address for hosting. 
We will have to check if the IP address is available and if not choose a different one. The Rust Standard library has examples of other functions like how to send and receieve data. 


## Midterm Goals

* Basic Block Building (Can click to buy block, A and D to move block, W to rotate, and S to drop)
* One working attack type (Slingshot that launches straight out)
* 1 block variants (Wood block)
* Two players can join a lobby, interactions eventually reach other player
* Blocks can stack on top of each other.
...

## Final Goals

* 10%: 2 power-ups (2x money and 2x attack speed)
* 20%: Players can connect and play against each other with minimal lag
* 20%: Blocks stack, collide, and fall within our physics engine
* 10%: 3 block types (Bank Block generates money. Wood block which is cheaper, weighs less, and easier to break. Metal block which is more expensive, weighs more, harder to break).
* 10%: 2 attack types (Bullet bill like attack (Attacks will ignore its own teams buildings) and slingshot type (Can set attack angle and then it will automatically attack using that angle)).
* 10%: Earn money from building, destroying blocks, and through bank block.

## Stretch Goals

* 5% Wind System that will interact with towers, and projectiles. Wind system will be either a fan block that will have a force pushing your enemies tower and projectiles, or a random scripted event that will push towards you or your opponents tower. The wind system will also speed up projectiles that are going in the direction of the wind.
* 5% Forced vertical auto scroller (rising skyline) 
