#include <SoftwareSerial.h>

// Define the RX and TX pins for the LoRa module
int rxPin = 2;  // Connect to TX of LoRa module
int txPin = 3;  // Connect to RX of LoRa module

// Initialize software serial for the LoRa module
SoftwareSerial LoRaSerial(rxPin, txPin);

void setup() {
  // Open serial communications and wait for port to open:
  Serial.begin(9600);
  // Set up the software serial port for the LoRa module
  LoRaSerial.begin(9600);

  Serial.println("Serial communication with LoRa module started. Type AT commands:");
}

void loop() {
  static String inputString = "";         // A String to hold incoming data
  static boolean stringComplete = false;  // Whether the string is complete

  if (LoRaSerial.available()) {
    char inChar = (char)LoRaSerial.read();

    // Add the incoming char to the string:
    inputString += inChar;
    // If the incoming character is a newline, set the string as complete
    if (inChar == '\n') {
      stringComplete = true;
    }
  }

  if (stringComplete) {
    if (inputString[1] == 'R' && inputString[2] == 'C') {
      // Split the string into parts
      int index = 0;
      String parts[3] = { "", "", "" };
      for (int i = 0; i < 3; i++) {
        int nextIndex = inputString.indexOf(',', index);
        if (nextIndex == -1) nextIndex = inputString.length();
        parts[i] = inputString.substring(index, nextIndex);
        index = nextIndex + 1;
      }
      parts[0] = "AT+SEND=0";

      // Then, reassemble the string
      String modifiedString = "";
      for (int i = 0; i < 3; i++) {  // Adjust the size of this loop as needed
        modifiedString += parts[i];
        if (i < 2) {
          modifiedString += ',';  // Don't add a comma after the last part
        }
      }
      modifiedString.trim();
      modifiedString += "\r\n";
      // Convert the modified string to a char array and send it to the LoRa module
      char charBuf[modifiedString.length() + 1];
      modifiedString.toCharArray(charBuf, modifiedString.length() + 1);
      LoRaSerial.write(charBuf, modifiedString.length());
      // Clear the inputString and reset stringComplete for the next input
    }
    inputString = "";
    stringComplete = false;
  }
}
