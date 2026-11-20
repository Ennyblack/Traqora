# Booking API Documentation

## get_booking

Retrieves booking details for a given booking ID. 

### Authorization
- Returns booking details ONLY to the booking owner (passenger) or the explicitly authorized operator (airline). 
- Panics with "Unauthorized booking access" if invoked by any other address.
