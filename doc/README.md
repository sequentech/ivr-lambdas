# Documentation

Sequent Telephone Voting system is defined in Amazon Connect.

Some general considerations:
- All the user inputs are configured by default with a timeout of 15 seconds.
- When an error happens for example authentication fails or the voter doesn't provide a valid answer or there's a timeout, we retry 3 times and after that tell him there was an error and end the call.

A bird's eye view of the telephone voting activity diagram is as follows:

![Activity Diagram: Bird's eye view](http://www.plantuml.com/plantuml/proxy?src=https://raw.githubusercontent.com/sequentech/ivr-lambdas/main/doc/activity-diagram-birds-eye-view.puml)

The first block is the authentication block:

![Activity Diagram: Authentication Block](http://www.plantuml.com/plantuml/proxy?src=https://raw.githubusercontent.com/sequentech/ivr-lambdas/main/doc/activity-diagram-authentication-block.puml)

<!--
We use plantuml diagrams as explained in https://github.com/Zingam/UML-in-Markdown
with images generated from files like:
![Class Diagram](http://www.plantuml.com/plantuml/proxy?src=https://raw.githubusercontent.com/sequentech/ivr-lambdas/master/doc/diagram-example.puml)
-->