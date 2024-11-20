(define (problem transport_p01_10_city_5nodes_1000size_3degree_100mindistance_2trucks_2packagespercity_2008seed-problem)
 (:domain transport_p01_10_city_5nodes_1000size_3degree_100mindistance_2trucks_2packagespercity_2008seed-domain)
 (:objects
   city_loc_1 city_loc_2 city_loc_3 city_loc_4 city_loc_5 - location
   truck_1 truck_2 - vehicle
   package_1 package_2 - package
 )
 (:init (road city_loc_3 city_loc_1) (= (road_length city_loc_3 city_loc_1) 22) (= (fuel_demand city_loc_3 city_loc_1) 43) (road city_loc_1 city_loc_3) (= (road_length city_loc_1 city_loc_3) 22) (= (fuel_demand city_loc_1 city_loc_3) 43) (road city_loc_3 city_loc_2) (= (road_length city_loc_3 city_loc_2) 50) (= (fuel_demand city_loc_3 city_loc_2) 99) (road city_loc_2 city_loc_3) (= (road_length city_loc_2 city_loc_3) 50) (= (fuel_demand city_loc_2 city_loc_3) 99) (road city_loc_4 city_loc_1) (= (road_length city_loc_4 city_loc_1) 26) (= (fuel_demand city_loc_4 city_loc_1) 52) (road city_loc_1 city_loc_4) (= (road_length city_loc_1 city_loc_4) 26) (= (fuel_demand city_loc_1 city_loc_4) 52) (road city_loc_4 city_loc_3) (= (road_length city_loc_4 city_loc_3) 45) (= (fuel_demand city_loc_4 city_loc_3) 89) (road city_loc_3 city_loc_4) (= (road_length city_loc_3 city_loc_4) 45) (= (fuel_demand city_loc_3 city_loc_4) 89) (road city_loc_5 city_loc_1) (= (road_length city_loc_5 city_loc_1) 37) (= (fuel_demand city_loc_5 city_loc_1) 74) (road city_loc_1 city_loc_5) (= (road_length city_loc_1 city_loc_5) 37) (= (fuel_demand city_loc_1 city_loc_5) 74) (road city_loc_5 city_loc_4) (= (road_length city_loc_5 city_loc_4) 12) (= (fuel_demand city_loc_5 city_loc_4) 24) (road city_loc_4 city_loc_5) (= (road_length city_loc_4 city_loc_5) 12) (= (fuel_demand city_loc_4 city_loc_5) 24) (at_ package_1 city_loc_3) (= (package_size package_1) 23) (at_ package_2 city_loc_4) (= (package_size package_2) 55) (has_petrol_station city_loc_1) (at_ truck_1 city_loc_3) (ready_loading truck_1) (= (capacity truck_1) 100) (= (fuel_left truck_1) 424) (= (fuel_max truck_1) 424) (at_ truck_2 city_loc_4) (ready_loading truck_2) (= (capacity truck_2) 100) (= (fuel_left truck_2) 424) (= (fuel_max truck_2) 424))
 (:goal (and (at_ package_1 city_loc_2) (at_ package_2 city_loc_3)))
 (:metric minimize (total-time))
)
