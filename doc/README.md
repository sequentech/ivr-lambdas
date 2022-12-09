# Documentation

Sequent Telephone Voting system is defined in Amazon Connect.

Some general considerations:
- All the user inputs are configured by default with a timeout of 15 seconds.
- When an error happens for example authentication fails or the voter doesn't provide a valid answer or there's a timeout, we retry 3 times and after that tell him there was an error and end the call.

A bird's eye view of the telephone voting activity diagram is as follows:

![Activity Diagram: Bird's eye view](http://www.plantuml.com/plantuml/proxy?src=https://raw.githubusercontent.com/sequentech/ivr-lambdas/main/doc/activity-diagram-birds-eye-view.puml)

## 1. Authentication Block

The first block is the authentication block:

![Activity Diagram: Authentication Block](http://www.plantuml.com/plantuml/proxy?src=https://raw.githubusercontent.com/sequentech/ivr-lambdas/main/doc/activity-diagram-authentication-block.puml)

The authentication block contains the following elements:

### AT-1: Welcome

- **Element type:** Play Prompt
- **Spoken text:** `Welcome: this is the telephone voting system for the  strike election of Ontario's Catholic Teachers Association.`
- **On Error:** Goes to `AT-4: Error Message` element.

### AT-2: Start Login

- **Element type:** Play Prompt
- **Spoken text:** `We will now request your authentication credentials.`
- **On Error:** Goes to `AT-4: Error Message` element.

### AT-3: Repeated loop 3 times?

- **Element type:** Loop Condition
- **Condition:** Check if this element has been entered more than 3 times and increase the counter by one.
- **Positive Result:** Goes to `AT-4: Error Message` element.
- **Negative Result:** Goes to `AT-5` Login Process block.

### AT-4: Error message

- **Element type:** Play Prompt
- **Spoken text:** `We're sorry, an error occurred. Please try again later. Goodbye.`
- **On Error:** Ends the call.

### AT-5: Login Process

- **Element type:** Block
- **Description:** This is block grouping the login process.

### AT-6: Register Membership Id

- **Element type:** Store Input
- **Description:** Stores numerical input to contact attribute. Plays an interruptible audio prompt and stores digits via DTMF as a contact attribute. Any invalid input will trigger the `On Error` action.
- **Spoken text:** `Please enter your Membership Number Id and press the pound key when complete.`
- **Timeout:** 15 seconds
- **On Timeout:** Goes to `AT-9: Auth Error` element.
- **On Error:** Goes to `AT-9: Auth Error` element.

### AT-7: Register PIN

- **Element type:** Store Input
- **Description:** Stores numerical input to contact attribute. Plays an interruptible audio prompt and stores digits via DTMF as a contact attribute. Any invalid input will trigger the `On Error` action.
- **Spoken text:** `Please enter your PIN number and press the pound key when complete.`
- **Timeout:** 15 seconds
- **On Timeout:** Goes to `AT-9: Auth Error` element.
- **On Error:** Goes to `AT-9: Auth Error` element.

### AT-8: Call Auth API

- **Element type:** Call API
- **Description:**  Calls the Sequent Platform's API to authenticate the voter with the provided credentials to obtain an authentication token.
- **Timeout:** 15 seconds
- **On Error or Timeout:** Goes to `AT-9: Auth Error` element.
- **Restrictions:** maximum 20 digits
- **On Error:** Goes to `AT-9: Auth Error` element.

### AT-9: Auth Error

- **Element type:** Play Prompt
- **Spoken text:** `Authentication unsuccessful. Please try again.`
- **On Error:** Go to `AT-2: Start Login` element.

### AT-10: Did Auth Succeed?

- **Element type:** Loop Condition
- **Condition:** Check if authentication was successful.
- **Positive result:** Goes to `AT-11 Auth Success` element.
- **Negative result:** Goes to `AT-9 Auth Error` element.

### AT-11: Auth Success

- **Element type:** Play Prompt
- **Spoken text:** `Great, you are now authenticated..`
- **On Error:** Go to `AT-4: Error Message` element.

<!--
We use plantuml diagrams as explained in https://github.com/Zingam/UML-in-Markdown
with images generated from files like:
![Class Diagram](http://www.plantuml.com/plantuml/proxy?src=https://raw.githubusercontent.com/sequentech/ivr-lambdas/master/doc/diagram-example.puml)
-->