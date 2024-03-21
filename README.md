# Squirrel

## What is ?

Squirrel is a project whihc aims to reduce the stress of attendees while registrating to a furry convention. 

At the moment, registration are a mess (Fauntastic is out of tickets under 32 seconds, FBL is about the same) and generates a lot of anxiety for people who didn't have the chance to pre-register (only the staff have the right to do so). It assures fairness by using a randomised place in the waiting list instead of the "first arrived first served" system, which is not fair against people which have a poor connection quality or personal issue (like, a job).

Squirrel provides a reactive front-end, to make attendees to just wait under a portal their turn to access the registration system.

## How does it work ?

- During the year, attendees can pre-register on the squirrel portal by creating an account. 
- Some minutes before registration, every user should be waiting over the portal
- When registration opens, the waiting list is computed randomly from the pre-registration list. Every attendee is assigned to a random place in the waiting list
- From now on, if an attendee refreshs the page using F5, it will be automatically put at the end of the waiting list
- The waiting list is depiled, from first to end, until the registration has no tickets left or the waiting list is empty
- When an attendee is depiled from the waiting list, they are automatically redirected to the registration system. They don't have anything to do on portal unless wait to be redirected !
- If they are redirected to the registration, there are tickets available for them !

## How does it work (technically)

- At pre-registration phase, each user will be given a PreRegistrationToken which assures them a place in the waiting list when registration starts
- Some minutes before the time of registration, attendees will connect to an SSE gateway through the portal and receive real-time events and messages about the registration status (number of tickets left for example)
- 