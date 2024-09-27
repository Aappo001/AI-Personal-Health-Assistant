const express = require('express');
const jwt = require('jsonwebtoken');
const bodyParser = require('body-parser');
const app = express();

app.use(bodyParser.json());

const mockUser = {
    username: 'testuser',
    password: 'testpassword'
};

// Login route (for simplicity)
app.post('/login', (req, res) => {
    const { username, password } = req.body;
    
    // Simple authentication check (replace with database check)
    if (username === mockUser.username && password === mockUser.password) {
        // Generate JWT token
        const token = jwt.sign({ username }, 'yourSecretKey', { expiresIn: '1h' });
        res.json({ token });
    } else {
        res.status(401).json({ error: 'Invalid credentials' });
    }
});

app.listen(3000, () => {
    console.log('Server running on port 3000');
});
