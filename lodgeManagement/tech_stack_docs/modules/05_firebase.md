# Technology: Firebase (Auth, Firestore, Cloud Storage)

**Firebase** is a comprehensive backend-as-a-service (BaaS) platform by Google.

## How It Helps the Lodge Management System
* **Real-time Synchronization**: Firestore database uses WebSockets to push live updates to the client. When a staff member checks a customer in or books a room from one device, the room's status updates in real-time across all active terminals without page refreshes.
* **Serverless Scale & Maintenance**: By leveraging Firebase, the project avoids managing, securing, and maintaining custom REST servers, relational databases, and OS environments.
* **Secure Google Authentication**: Firebase Auth provides secure OAuth workflows out-of-the-box, allowing staff members to sign in safely with their Google Workspace credentials.

## Specific Role in this Project
* **Database**: Holds the `rooms`, `customers`, and `bookings` collections, and lists authorized operators under `allowed_users`.
* **Storage**: Holds high-resolution uploaded images (such as guests' profile photos).
* **Auth**: Handles active operator login and session state validation.
