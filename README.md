# Tower Defense

# GAMENAMEHERE

by TEAMNAMEHERE

## Team Members
* Advanced Topic Subteam 1: Custom Physics Engine
	* STUDENT1_USERNAME_HERE: Ishay Wirthiem
 	* STUDENT2_USERNAME_HERE: Gio Bair
 	* brr154: Ben Rand

* Advanced Topic Subteam 2: Network Multiplayer
	* STUDENT1_USERNAME_HERE: Polly Naneva
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

**Structure Stability:** Towers lean, wobble, and topple over based on weight distribution and block type.

**Physics Driven Attacks:** impact and knockback forces push through connected blocks for realistic, readable damage.
    
### Network Multiplayer

**Host/Client Model:** One player acts as host and generates a room code; the second player joins the same match by entering it.

**Real Time State Sync:** Everything stays synchronized between clients like block placements, currency, and attacks

**Latency Handling:** Keeps game fair under lag


## Midterm Goals

* Basic Block Building
* One working attack type
* 2 block variants
* Two players can join a lobby, interactions eventually reach other player
* Blocks can stack
...

## Final Goals

* 10%: 2 power-ups
* 20%: Players can connect and play against each other with minimal lag
* 20%: Blocks stack, collide, and fall within our physics engine
* 10%: 3 block types
* 10%: 2 attack types
* 10%: Earn money from building, destroying blocks, and through bank block.
...

## Stretch Goals

* Wind System that will interact with towers, and projectiles
* Forced vertical auto scroller (rising skyline) 
