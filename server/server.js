const express = require('express');
const bodyParser = require('body-parser');
const cors = require('cors');
const fs = require('fs');
const moment = require('moment');

const app = express();
const PORT = 3000;


app.use(cors());
app.use(bodyParser.json());
app.use(express.static('public')); 

// Serve the login page
app.get('/', (req, res) => {
    res.sendFile(__dirname + '/login.html');
});

// Serve the registration page
app.get('/register', (req, res) => {
    res.sendFile(__dirname + '/register.html');
});

app.get('/users', (req, res) => {
    const usersWithoutPasswords = users.map(({ password, ...user }) => user);
    return res.status(200).json(usersWithoutPasswords);
});


const users = [];

// Login route
app.post('/login', (req, res) => {
    const { username, password } = req.body;

    // Find the user in the mock database
    const user = users.find(u => u.username === username);

    if (user && user.password === password) {
    
        logLoginAttempt(username, true);
        return res.status(200).json({ message: 'Login successful!' });
    } else {
        
        logLoginAttempt(username, false);
        return res.status(401).json({ message: 'Invalid username or password.' });
    }
});

// Registration route
app.post('/register', (req, res) => {
    const { firstName, lastName, username, email, password } = req.body;


    // Check if username already exists
    const existingUser = users.find(u => u.username === username);
    if (existingUser) {
        return res.status(400).json({ message: 'Username already exists.' });
    }


    // Add new user 
    users.push({ firstName, lastName, username, email, password });


    return res.status(201).json({ message: 'Registration successful!' });

});

// Function to log login attempts
function logLoginAttempt(username, success) {
    const timestamp = moment().format('YYYY-MM-DD HH:mm:ss');
    const logMessage = `${timestamp} - Login - Username: ${username}, Success: ${success}\n`;

    // Append log message to a log file
    fs.appendFile('login_attempts.log', logMessage, (err) => {
        if (err) {
            console.error('Error writing to log file', err);
        }
    });
}


// Start the server
app.listen(PORT, () => {
    console.log(`Server is running on http://localhost:${PORT}`);
});
