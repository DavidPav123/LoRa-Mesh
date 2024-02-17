#include <SoftwareSerial.h>

// Define the RX and TX pins for the LoRa module
int rxPin = 2; // Connect to TX of LoRa module
int txPin = 3; // Connect to RX of LoRa module

// Initialize software serial for the LoRa module
SoftwareSerial LoRaSerial(rxPin, txPin);

void setup() {
  // Open serial communications and wait for port to open:
  Serial.begin(57600);
  // Set up the software serial port for the LoRa module
  LoRaSerial.begin(57600);

  Serial.println("Serial communication with LoRa module started. Type AT commands:");
}

void loop() {
  static String inputString = ""; // A String to hold incoming data
  static boolean stringComplete = false; // Whether the string is complete

  if (Serial.available()) {
    char inChar = (char)Serial.read();
    // Add the incoming char to the string:
    inputString += inChar;
    // If the incoming character is a newline, set the string as complete
    if (inChar == '\n') {
      stringComplete = true;
    }
  }

  // If the string is complete, convert it to a char array and send it to the LoRa module
  if (stringComplete) {
    // Allocate a buffer for the character array. Add 1 for the null terminator.
    char charBuf[inputString.length() + 1];
    // Convert the String to a char array
    inputString.toCharArray(charBuf, inputString.length() + 1);
    // Send the char array to the LoRa module
    LoRaSerial.write(charBuf, inputString.length());

    // Optionally, clear the inputString and reset stringComplete for the next input
    inputString = ""; // Clear the string for new input
    stringComplete = false; // Reset the flag
  }

  // If data is available from the LoRa module, send it back to the computer
  if (LoRaSerial.available()) {
    Serial.write(LoRaSerial.read());
  }
}
