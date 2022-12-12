# Documentation

Sequent Telephone Voting system is defined in Amazon Connect.

Some general considerations:
- All the user inputs are configured by default with a timeout of 15 seconds.
- When an error happens for example authentication fails or the voter doesn't provide a valid answer or there's a timeout, we retry 3 times and after that tell him there was an error and end the call.

A bird's eye view of the telephone voting activity diagram is as follows:

![Activity Diagram: Bird's eye view](http://www.plantuml.com/plantuml/proxy?src=https://raw.githubusercontent.com/sequentech/ivr-lambdas/main/doc/activity-diagram-birds-eye-view.puml)

## 1. B-1: Authentication Block

The first block is the authentication block:

![Activity Diagram B-1: Authentication Block](http://www.plantuml.com/plantuml/proxy?src=https://raw.githubusercontent.com/sequentech/ivr-lambdas/main/doc/activity-diagram-authentication-block.puml)

The authentication block contains the following elements:

### AT-1: Welcome

- **Element type:** Play Prompt
- **Spoken text:** `Welcome: this is the telephone voting system for the  strike election of Ontario's Catholic Teachers Association.`
- **On Error:** Goes to [AT-3: Error Message](#at-3-error-message) element.

### AT-2: Repeated Login 3 Times?

- **Element type:** Conditional Branch
- **Condition:** Check if this element has been entered more than 3 times and increase the counter by one.
- **Positive Result:** Goes to [AT-3: Error Message](#at-3-error-message) element.
- **Negative Result:** Goes to [AT-4: Login Process](#at-4-login-process) block.

### AT-3: Error message

- **Element type:** Play Prompt
- **Spoken text:** `We're sorry, an error occurred. Please try again later. Goodbye.`
- **On Error:** Ends the call.

### AT-4: Login Process

- **Element type:** Block
- **Description:** This is block grouping the login process.

### AT-5: Start Login

- **Element type:** Play Prompt
- **Spoken text:** `We will now request your authentication credentials.`
- **On Error:** Goes to [AT-3: Error Message](#at-3-error-message) element.

### AT-6: Register Membership Id

- **Element type:** Store Input
- **Description:** Stores numerical input to contact attribute. Plays an interruptible audio prompt and stores digits via DTMF as a contact attribute. Any invalid input will trigger the `On Error` action.
- **Spoken text:** `Please enter your Membership Number Id and press the pound key when complete.`
- **Timeout:** 15 seconds
- **On Timeout:** Goes to [AT-9: Auth Error](#at-9-auth-error) element.
- **Restrictions:** Maximum 20 digits
- **On Error:** Goes to [AT-9: Auth Error](#at-9-auth-error) element.

### AT-7: Register PIN

- **Element type:** Store Input
- **Description:** Stores numerical input to contact attribute. Plays an interruptible audio prompt and stores digits via DTMF as a contact attribute. Any invalid input will trigger the `On Error` action.
- **Spoken text:** `Please enter your PIN number and press the pound key when complete.`
- **Timeout:** 15 seconds
- **On Timeout:** Goes to [AT-9: Auth Error](#at-9-auth-error) element.
- **Restrictions:** Maximum 20 digits
- **On Error:** Goes to [AT-9: Auth Error](#at-9-auth-error) element.

### AT-8: Call Auth API

- **Element type:** Call API
- **Description:**  Calls the Sequent Platform's API to authenticate the voter with the provided credentials to obtain an authentication token.
- **Timeout:** 15 seconds
- **On Error or Timeout:** Goes to [AT-9: Auth Error](#at-9-auth-error) element.

### AT-9: Auth Error

- **Element type:** Play Prompt
- **Spoken text:** `Authentication unsuccessful. Please try again.`
- **On Error:** Go to [AT-2: Repeated Login 3 Times?](#at-2-repeated-login-3-times) element.

### AT-10: Did Auth Succeed?

- **Element type:** Conditional Branch
- **Condition:** Check if authentication was successful.
- **Positive result:** Goes to [AT-11 Auth Success](#at-11-auth-success) element.
- **Negative result:** Goes to [AT-9 Auth Error](#at-9-auth-error) element.

### AT-11: Auth Success

- **Element type:** Play Prompt
- **Spoken text:** `Great, you are now authenticated..`
- **On Error:** Go to [AT-3: Error Message](#at-3-error-message) element.

### AT-12: Continue to B-2: Voting Block

- **Element type:** No-op
- **Description:** This is not an actual element, it only represents the continuation to the [Voting Block](#2-b-2-voting-block).


## 2. B-2: Voting Block

The diagram for the Voting Block is the following:

![Activity Diagram B-2: Voting Block](http://www.plantuml.com/plantuml/proxy?src=https://raw.githubusercontent.com/sequentech/ivr-lambdas/main/doc/activity-diagram-voting-block.puml)

This block contains the following elements:

### VT-1: Start Voting

- **Element type:** Play Prompt
- **Spoken text:** `Follow the steps to choose your options and cast your ballot. The system will now process your vote.`
- **On Error:** Goes to [VT-3: Error Message](#vt-3-error-message) element.

### VT-2: Repeated Loop 3 Times?

- **Element type:** Conditional Branch
- **Condition:** Check if this element has been entered more than 3 times and increase the counter by one.
- **Positive Result:** Goes to [VT-3: Error Message](#vt-3-error-message) element.
- **Negative Result:** Goes to [VT-4: Choose Vote](#vt-4-choose-vote) block.

### VT-3: Error message

- **Element type:** Play Prompt
- **Spoken text:** `We're sorry, an error occurred. Please try again later. Goodbye.`
- **On Error:** Ends the call.

### VT-4: Choose Vote

- **Element type:** Store Option Input
- **Description:** Delivers an audio message to solicit customer input. Based on response, the contact flow branches. Any invalid response is considered an error.
- **Spoken text:** `The question to vote is: Do you approve the strike action document? Press 1 to vote Yes. Press 2 to vote No.`
- **On Press 1**: Record selection and goes to [VT-5: Confirm Vote](#vt-5-confirm-vote)
- **On Press 2**: Record selection and goes to [VT-5: Confirm Vote](#vt-5-confirm-vote)
- **On Error:** Goes to [VT-6: Voting Error](#vt-9-voting-error) element.
- **Timeout:** 15 seconds
- **On Timeout:** Goes to [VT-6: Voting Error](#vt-9-voting-error) element.

### VT-5: Confirm Vote

- **Element type:** Store Option Input
- **Description:** Delivers an audio message to solicit customer input. Based on response, the contact flow branches. Any invalid response is considered an error.
- **Spoken text:** `You chose to vote ${vote}. Press 1 to confirm this is what you want to vote. Press 2 to listen again the question and change your vote.`
- **On Press 1 or 2**: Positively confirms selection and goes to [VT-7: Selection Confirmed?](#vt-7-selection-confirmed)
- **On Error:** Negatively confirms selection and goes to [VT-7: Selection Confirmed?](#vt-7-selection-confirmed)(#vt-9-voting-error) element.
- **Timeout:** 15 seconds
- **On Timeout:** Goes to [VT-6: Voting Error](#vt-9-voting-error) element.

### VT-7: Selection Confirmed?

- **Element type:** Conditional Branch
- **Condition:** Check if selection was confirmed.
- **Positive Result:** Goes to [VT-8: Error Message](#vt-3-error-message) element.
- **Negative Result:** Goes to [VT-4: Choose Vote](#vt-4-choose-vote) block.


### VT-8: Continue to B-3: Casting Block

- **Element type:** No-op
- **Description:** This is not an actual element, it only represents the continuation to the [Casting Block](#3-b-3-casting-block).


## 3. B-3: Casting Block

The diagram for the Casting Block is the following:

![Activity Diagram B-3: Casting Block](http://www.plantuml.com/plantuml/proxy?src=https://raw.githubusercontent.com/sequentech/ivr-lambdas/main/doc/activity-diagram-casting-block.puml)

This block contains the following elements:

### CT-1: Call Cast Vote API

- **Element type:** Call API
- **Description:**  Encrypts the vote and calls the Sequent Platform's API to cast the encrypted vote the voter with the authentication token.
- **Timeout:** 15 seconds
- **On Error or Timeout:** Goes to [CT-8: Vote Error](#ct-8-vote-error) element.

### CT-2: Did Vote Casting Succeed?

- **Element type:** Conditional Branch
- **Condition:** Check if the vote casting was successful.
- **Positive Result:** Goes to [CT-3: Vote Cast Successful](#ct-3-vote-cast-successful) element.
- **Negative Result:** Goes to [CT-8: Vote Error](#ct-8-vote-error) element.

### CT-3: Vote Cast Successful

- **Element type:** Play Prompt
- **Spoken text:** `Your vote was cast successfully.`
- **On Error:** Goes to [CT-8: Vote Error](#ct-8-vote-error) element.

### CT-4: Repeated Loop 3 Times?

- **Element type:** Conditional Branch
- **Condition:** Check if this element has been entered more than 3 times.
- **Positive Result:** Goes to [CT-5: Thanks Goodbye](#ct-5-thanks-goodbye) element.
- **Negative Result:** Goes to [CT-6: Listen Ballot Receipt](#ct-6-listen-ballot-receipt) element.

### CT-5: Thanks Goodbye

- **Element type:** Play Prompt
- **Spoken text:** `Thank you for voting in this election. Have a nice day. Goodbye.`
- **On Error:** Ends the call.

### CT-6: Listen Ballot Receipt

- **Element type:** Store Option Input
- **Description:** Delivers an audio message to solicit customer input. Based on response, the contact flow branches. Any invalid response is considered an error.
- **Spoken text:** `Your ballot receipt-id is ${ReceiptId}. Press 1 to finish. Press 2 to hear the ballot receipt-id again.`
- **On Press 1:** Set Listen to Ballot Receipt Again to `false` and goes to [CT-7: Listen Ballot Receipt Again?](#ct-7-listen-ballot-receipt-again)
- **On Press 2:** Set Listen to Ballot Receipt Again to `true` and goes to [CT-7: Listen Ballot Receipt Again?](#ct-7-listen-ballot-receipt-again)
- **On Error:** Set Listen to Ballot Receipt Again to `true` and goes to [CT-7: Listen Ballot Receipt Again?](#ct-7-listen-ballot-receipt-again)
- **Timeout:** 15 seconds
- **On Timeout:** Set Listen to Ballot Receipt Again to `true` and goes to [CT-7: Listen Ballot Receipt Again?](#ct-7-listen-ballot-receipt-again)

### CT-7: Listen Ballot Receipt Again?

- **Element type:** Conditional Branch
- **Condition:** Check if Listen to Ballot Receipt Again was set to `true`
- **Positive Result:** Goes to [CT-4: Repeated Loop 3 Times?](#ct-4-repeated-loop-3-times) element.
- **Negative Result:** Goes to [CT-5: Thanks Goodbye](#ct-5-thanks-goodbye) element.

### CT-8: Vote Error

- **Element type:** Play Prompt
- **Spoken text:** `We're sorry, an error occurred. Please try again later. Goodbye.`
- **On Error:** Ends the call.
