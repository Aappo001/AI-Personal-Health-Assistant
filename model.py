import re
import numpy as np
import sqlite3
from collections import Counter
from sklearn.model_selection import train_test_split
from sklearn.naive_bayes import MultinomialNB
from sklearn.metrics import accuracy_score, classification_report

# Sample stopwords (can be expanded)
STOPWORDS = {"the", "is", "and", "in", "to", "on", "it", "with"}

# Manual tokenization
def tokenize(text):
    # Convert text to lowercase and split into words (tokens)
    tokens = re.findall(r'\b\w+\b', text.lower())
    return [token for token in tokens if token not in STOPWORDS]

# Manual symptom extraction
def extract_symptoms(user_input, symptom_keywords):
    tokens = tokenize(user_input)
    symptoms = [token for token in tokens if token in symptom_keywords]
    return symptoms

#Symptom matching (not 100% accurate obviously)
# Known symptoms (could be expanded)
known_symptoms = ["headache", "fever", "nausea", "cough", "dizziness", "fatigue", "pain", "vomiting"]

# Sample data (a simple mapping from symptoms to diagnoses)
data = [
    {"symptoms": ["headache", "fever"], "diagnosis": "flu"},
    {"symptoms": ["cough", "fever"], "diagnosis": "cold"},
    {"symptoms": ["nausea", "vomiting"], "diagnosis": "food poisoning"},
    {"symptoms": ["dizziness", "fatigue"], "diagnosis": "anemia"},
    {"symptoms": ["headache", "fatigue"], "diagnosis": "migraine"},
]

# Extract symptoms from user input and find diagnosis
def find_diagnosis(user_symptoms):
    # Compare extracted symptoms to each entry in the dataset
    for entry in data:
        # If all symptoms match, return the diagnosis
        if set(user_symptoms).issubset(set(entry["symptoms"])):
            return entry["diagnosis"]
    return "No diagnosis found. Consider consulting a doctor."

# Example usage
user_input = "I have a headache and fever"
symptoms = extract_symptoms(user_input, known_symptoms)
diagnosis = find_diagnosis(symptoms)
print(f"Diagnosis: {diagnosis}")

#below is a basic Naive Bayes classifier that we can use for symptom diagnosis 
symptom_data = [
    "headache fever", 
    "cough fever", 
    "nausea vomiting", 
    "dizziness fatigue", 
    "headache fatigue"
]
diagnoses = ["flu", "cold", "food poisoning", "anemia", "migraine"]

# Tokenize the symptom data and create a vocabulary
def create_vocabulary(data):
    vocab = set()
    for entry in data:
        tokens = tokenize(entry)
        vocab.update(tokens)
    return list(vocab)

# Convert text data to a feature matrix (binary vector for presence of words)
def text_to_features(data, vocab):
    features = []
    for entry in data:
        tokens = tokenize(entry)
        # Binary vector representing presence of each word in vocab
        feature_vector = [1 if word in tokens else 0 for word in vocab]
        features.append(feature_vector)
    return np.array(features)

# Create the vocabulary and transform data
vocab = create_vocabulary(symptom_data)
X = text_to_features(symptom_data, vocab)
y = np.array(diagnoses)

# Split data into training and test sets
X_train, X_test, y_train, y_test = train_test_split(X, y, test_size=0.2, random_state=42)

# Train a Naive Bayes classifier
model = MultinomialNB()
model.fit(X_train, y_train)

# Evaluate the model
y_pred = model.predict(X_test)
print(f'Accuracy: {accuracy_score(y_test, y_pred) * 100:.2f}%')
print(classification_report(y_test, y_pred))

# Function to predict based on new user input
def predict_diagnosis(user_input):
    user_symptoms = tokenize(user_input)
    feature_vector = [1 if word in user_symptoms else 0 for word in vocab]
    prediction = model.predict([feature_vector])
    return prediction[0]

# Example usage
user_input = "headache and fatigue"
diagnosis = predict_diagnosis(user_input)
print(f'Predicted diagnosis: {diagnosis}')

#Storing data idk if this is already in another branch but ill add it here too
# Create SQLite database connection
conn = sqlite3.connect('health_assistant.db')
cursor = conn.cursor()

# Create a table to store interactions
cursor.execute('''
CREATE TABLE IF NOT EXISTS interactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_input TEXT,
    model_response TEXT,
    feedback TEXT
)
''')
conn.commit()

# Function to store the users interactions
def store_interaction(user_input, response, feedback=None):
    cursor.execute('''
        INSERT INTO interactions (user_input, model_response, feedback)
        VALUES (?, ?, ?)
    ''', (user_input, response, feedback))
    conn.commit()

# Function to retrieve previous interactions
def retrieve_previous_interaction(user_input):
    cursor.execute('SELECT model_response FROM interactions WHERE user_input=?', (user_input,))
    result = cursor.fetchone()
    return result[0] if result else None

#
def ask_for_feedback(user_input, response):
    print(f"AI Response: {response}")
    feedback = input("Did you find this response helpful? (yes/no): ").strip().lower()
    store_interaction(user_input, response, feedback)

def run_health_assistant():
    while True:
        user_input = input("Hello please describe your symptoms so I can help: ")
        
        # Check if a similar input exists in the database
        previous_response = retrieve_previous_interaction(user_input)
        
        if previous_response:
            print(f"Previous Response: {previous_response}")
            ask_for_feedback(user_input, previous_response)
        else:
            # Extract symptoms and find diagnosis
            symptoms = extract_symptoms(user_input, known_symptoms)
            diagnosis = find_diagnosis(symptoms)
            print(f"Diagnosis: {diagnosis}")
            ask_for_feedback(user_input, diagnosis)


run_health_assistant()
