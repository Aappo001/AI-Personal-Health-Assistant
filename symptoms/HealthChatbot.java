
import java.util.ArrayList;
import java.util.List;
import java.util.Scanner;

public class HealthChatbot {
    public static void main(String[] args) {
        List<Symptom> symptoms = new ArrayList<>();
        
        // Create symptoms and add solutions
        Symptom headache = new Symptom("headache");
        headache.addSolution("mild", "Take Advil.");
        headache.addSolution("medium", "Take Tylenol and rest.");
        headache.addSolution("severe", "Consult a doctor immediately.");
        symptoms.add(headache);

        Symptom fever = new Symptom("fever");
        fever.addSolution("mild", "Drink fluids and rest.");
        fever.addSolution("medium", "Take ibuprofen and monitor temperature.");
        fever.addSolution("severe", "Seek medical attention.");
        symptoms.add(fever);

        Symptom stomachAche = new Symptom("stomach ache");
        stomachAche.addSolution("mild", "Drink ginger tea.");
        stomachAche.addSolution("medium", "Take antacids and rest.");
        stomachAche.addSolution("severe", "Consult a doctor if pain persists.");
        symptoms.add(stomachAche);

        Symptom drowziness = new Symptom("drowziness");
        drowziness.addSolution("mild", "Take a small nap.");
        drowziness.addSolution("medium", "Slowly rub your temples.");
        drowziness.addSolution("severe", "Get a cup of coffee.");
        symptoms.add(drowziness);

        Symptom anxiety = new Symptom("anxiety");
        anxiety.addSolution("mild", "Write your worries in a journal.");
        anxiety.addSolution("medium", "Drink a warm cup of herbal tea.");
        anxiety.addSolution("severe", "Loosen your muscles, sit down, and try to relax.");
        symptoms.add(drowziness);

        Symptom stress = new Symptom("stress");
        stress.addSolution("mild", "Prioritize your tasks from most important to least important");
        stress.addSolution("medium", "Take an hour to do activties you enjoy");
        stress.addSolution("severe", "Go outside, get some fresh air, and clear your mind");
        symptoms.add(stress);
                
        Scanner scanner = new Scanner(System.in);
        
        // Display symptom choices from the list
        System.out.println("Please choose a symptom from the following options:");
        for (Symptom symptom : symptoms) {
            System.out.println("- " + symptom.getName());
        }

        // Prompt user for symptom choice and validate input
        Symptom selectedSymptom = null;
        while (true) {
            System.out.println("Enter your symptom:");
            String symptomChoice = scanner.nextLine().toLowerCase();

            for (Symptom symptom : symptoms) {
                if (symptom.getName().equals(symptomChoice)) {
                    selectedSymptom = symptom;
                    break;
                }
            }

            if (selectedSymptom != null) {
                break;  // Valid symptom, break out of the loop
            } else {
                System.out.println("Invalid symptom. Please choose one from the list.");
            }
        }

        // Display severity choices
        System.out.println("How severe is your " + selectedSymptom.getName() + "? (mild/medium/severe)");

        // Prompt user for severity level and validate input
        String severity;
        while (true) {
            severity = scanner.nextLine().toLowerCase();
            if (selectedSymptom.getSolution(severity) != null) {
                break;  // Valid severity, break out of the loop
            } else {
                System.out.println("Invalid severity. Please enter mild, medium, or severe.");
            }
        }

        // Provide the appropriate solution
        String solution = selectedSymptom.getSolution(severity);
        System.out.println("Suggested treatment: " + solution);

        scanner.close();
    }
}
