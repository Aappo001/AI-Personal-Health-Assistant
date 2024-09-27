const express = require('express');
const bcrypt = require('bcryptjs');
const jwt = require('jsonwebtoken');
const bodyParser = require('body-parser');
const db = require('./database'); // SQLite connection

const app = express();
app.use(bodyParser.json());

// Register new user
app.post('/register', async (req, res) => {
    const { username, password } = req.body;

    // Hash the password
    const salt = await bcrypt.genSalt(10);
    const hashedPassword = await bcrypt.hash(password, salt);

    // Save to the database
    db.run('INSERT INTO users (username, password) VALUES (?, ?)', [username, hashedPassword], function (err) {
        if (err) {
            return res.status(500).send({ message: 'Error registering user.' });
        }
        const token = jwt.sign({ id: this.lastID }, 'your_secret_key');
        res.status(201).send({ message: 'User registered!', token });
    });
});

// Start the server
app.listen(3000, () => {
    console.log('Server running on http://localhost:3000');
});
